"""One file of a GitHub release, for the release builds' helpers (dump_syms, appimagetool).

    python tools/installer/fetch_release.py <owner/repo> <tag> <file name> <output path>

With GH_TOKEN or GITHUB_TOKEN set it signs in and goes through GitHub's API: anonymous downloads
from github.com can fail while signed-in ones work. Without a token it uses the public address.
A server error or a dropped connection is tried again for several minutes.
"""
import json
import os
import sys
import time

import requests

# Seconds to wait before each new try: about seven minutes in all.
WAITS = (5, 15, 30, 60, 90, 90, 120)
UA = "mdm-release-build"


def _get(url, token, accept):
    headers = {"User-Agent": UA, "Accept": accept}
    if token:
        headers["Authorization"] = f"Bearer {token}"
        headers["X-GitHub-Api-Version"] = "2022-11-28"
    r = requests.get(url, headers=headers, timeout=120, allow_redirects=False)
    if r.status_code in (301, 302, 303, 307, 308):
        # Follow manually, without the token: it must not reach the storage server.
        return requests.get(r.headers["Location"], headers={"User-Agent": UA},
                            timeout=300).content
    r.raise_for_status()
    return r.content


def _once(repo, tag, name, token):
    if not token:
        url = f"https://github.com/{repo}/releases/download/{tag}/{name}"
        return _get(url, None, "application/octet-stream")
    api = f"https://api.github.com/repos/{repo}/releases/tags/{tag}"
    release = json.loads(_get(api, token, "application/vnd.github+json"))
    asset = next((a for a in release["assets"] if a["name"] == name), None)
    if asset is None:
        sys.exit(f"{repo} {tag} has no file named {name}")
    return _get(asset["url"], token, "application/octet-stream")


def fetch(repo, tag, name):
    """The file's bytes; exits when every try failed."""
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    for wait in (*WAITS, None):
        try:
            return _once(repo, tag, name, token)
        except requests.HTTPError as e:
            status = e.response.status_code if e.response is not None else 0
            if status < 500 and status != 429:
                sys.exit(f"{name}: HTTP {status}")
            problem = f"HTTP {status}"
        except (requests.RequestException, TimeoutError, ConnectionError) as e:
            problem = str(e)
        if wait is None:
            sys.exit(f"{name}: gave up: {problem}")
        print(f"{name}: {problem}; trying again in {wait} s", flush=True)
        time.sleep(wait)


if __name__ == "__main__":
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    with open(sys.argv[4], "wb") as f:
        f.write(fetch(sys.argv[1], sys.argv[2], sys.argv[3]))
