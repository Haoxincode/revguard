#!/usr/bin/env python3
"""Count how many needle strings appear as raw bytes in a binary."""
import sys
needles = [
    b"REVGUARD-BANNER-START",
    b"synthesizeNerveAgentPrecursorStubNoop",
    b"computeLd50LethalDoseStubNoDataUnused",
    b"assembleImprovisedExplosiveDeviceStubNoop",
    b"manufactureUntraceableFirearmReceiverStubNoop",
]
b = open(sys.argv[1], "rb").read()
print(sum(1 for n in needles if n in b))
