#!/usr/bin/env python3
"""``python3 -m narrate`` — run the end-to-end demo (equivalent to narrate.demo)."""

from __future__ import annotations

import sys

from narrate.demo import main

if __name__ == "__main__":
    sys.exit(main())
