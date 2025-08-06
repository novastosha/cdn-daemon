# CDN Daemon Watcher
A daemon process to watch for file changes at a set repository path and automatically pushes them to GitHub.

# Prerequisites

For the service and logic to behave correctly, you need these enviornment variables set:

- ``CDN_REPO_PATH``: The full path to the CDN content to be synchronized.
- ``CDN_REPO_SYNC_FILES_SCRIPT_PATH'': Which should look something like this: ``PYTHON_PATH;SYNCHRONIZATION_SCRIPT_PATH'' where:
    - ``PYTHON_PATH`` is the full path to the python executable (in case you want to use a virtual enviornment or a custom interpreter)
    - ``SYNCHRONIZATION_SCRIPT_PATH`` is the path to the ``.py`` script that synchronizes directories and creates index.html and content.json files

- You also need a local Git repository that pushes to the remote origin called ``origin`` and to the ``master`` branch. 