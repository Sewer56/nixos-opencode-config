#!/usr/bin/env python3
"""Cross-process agent semaphore using flock on slot files.

Each slot file = 1 agent capacity unit. Bulk acquire grabs N slot files.
flock is kernel-level: atomic, auto-released on process death (even SIGKILL).
No PID tracking, no sidecar, no watchdog — crash recovery is free.
"""

from __future__ import annotations

import fcntl
import os
import time
from pathlib import Path


class AgentSemaphore:
    def __init__(
        self,
        namespace: str,
        max_agents: int,
        slot_dir: str = "/tmp/opencode-agent-slots",
        timeout: float = 3600,
    ):
        self._dir = Path(slot_dir) / namespace
        self._max_agents = max_agents
        self._timeout = timeout
        self._held: list[tuple[int, int]] = []  # (fd, slot_index)

    def acquire(self, count: int = 1) -> None:
        self._dir.mkdir(parents=True, exist_ok=True)
        for i in range(self._max_agents):
            (self._dir / f"slot-{i}").touch(exist_ok=True)

        held_indices: set[int] = {idx for _, idx in self._held}
        deadline = time.time() + self._timeout

        while len(self._held) < count:
            for i in range(self._max_agents):
                if i in held_indices:
                    continue
                try:
                    fd = os.open(str(self._dir / f"slot-{i}"), os.O_RDONLY)
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self._held.append((fd, i))
                    held_indices.add(i)
                    if len(self._held) == count:
                        return
                except (OSError, IOError):
                    os.close(fd)
            if time.time() > deadline:
                self._release_held()
                raise TimeoutError(
                    f"AgentSemaphore({self._dir}): timeout after {self._timeout}s "
                    f"(needed {count}, got {len(self._held)})"
                )
            time.sleep(0.5)

    def release(self) -> None:
        self._release_held()

    def _release_held(self) -> None:
        for fd, _ in self._held:
            try:
                fcntl.flock(fd, fcntl.LOCK_UN)
            except (OSError, IOError):
                pass
            try:
                os.close(fd)
            except (OSError, IOError):
                pass
        self._held.clear()
