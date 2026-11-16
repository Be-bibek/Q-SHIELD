#!/usr/bin/env python3
"""
Q-SHIELD End-to-End Demonstration CLI Runner.
Executes the full pipeline:
Image Loading -> Preprocessing & Tiling -> Merkle Tree -> Manifest Creation
-> ML-DSA-65 Signature -> ML-KEM-768 Shared Secret -> HKDF-SHA-256 Session Key
-> AEAD Tile Encryption -> Simulated Interrupted Transmission -> Resumption
-> Cryptographic Verification -> 100% Lossless Image Reconstruction
"""

import subprocess
import sys

def main():
    print("=======================================================")
    print("      LAUNCHING Q-SHIELD FULL END-TO-END DEMO          ")
    print("=======================================================\n")
    
    cmd = ["cargo", "test", "-p", "qshield-transfer", "--test", "demo_e2e", "--", "--nocapture"]
    result = subprocess.run(cmd)
    
    if result.returncode == 0:
        print("\n[+] Q-SHIELD End-to-End Demonstration PASSED Successfully!")
        sys.exit(0)
    else:
        print("\n[-] Demonstration encountered an error.")
        sys.exit(result.returncode)

if __name__ == "__main__":
    main()
