#!/bin/bash
# entrypoint.sh for PA1 -- picks the right identity for the bind mount,
# so students never have to configure UIDs themselves.
#
# The container starts as root, looks at who owns /workspace, and then:
#   - owner is UID 0  -> Docker Desktop, rootless Docker, or Podman. In
#                        those setups container root *is* the host user,
#                        so we simply stay root.
#   - any other owner -> regular (rootful) Docker on Linux. We renumber the
#                        `student` user to match the host owner and drop
#                        to it, so files created in /workspace belong to
#                        the student on the host, not to root.
set -e

# Already non-root (e.g. someone passed --user): nothing to do.
if [ "$(id -u)" != "0" ]; then
    exec "$@"
fi

uid=$(stat -c %u /workspace)
gid=$(stat -c %g /workspace)

if [ "$uid" = "0" ]; then
    exec "$@"
fi

if [ "$uid" != "$(id -u student)" ] || [ "$gid" != "$(id -g student)" ]; then
    # -o allows IDs that collide with an existing system group/user
    # (e.g. macOS's "staff" group is GID 20).
    groupmod -o -g "$gid" student
    usermod  -o -u "$uid" -g "$gid" student
fi

export HOME=/home/student USER=student LOGNAME=student
exec setpriv --reuid=student --regid=student --init-groups "$@"
