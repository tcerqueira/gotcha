export interface PowChallenge {
  nonce: number;
  difficulty: number;
  timestamp: number;
}

export class ProofOfWork {
  private static readonly HASH_ALGORITHM = "SHA-256";

  private static async hashSolution(
    challenge: PowChallenge,
    solution: number,
  ): Promise<string> {
    const data = new Uint8Array([
      ...new Uint8Array(new BigUint64Array([BigInt(challenge.nonce)]).buffer), // u64
      ...new Uint8Array(new Uint16Array([challenge.difficulty]).buffer), // u16
      ...new Uint8Array(
        new BigInt64Array([BigInt(challenge.timestamp)]).buffer,
      ), // i64
      ...new Uint8Array(new BigUint64Array([BigInt(solution)]).buffer), // u64
    ]);

    const hashBuffer = await crypto.subtle.digest(this.HASH_ALGORITHM, data);
    return Array.from(new Uint8Array(hashBuffer))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }

  private static isSolution(hash: string, difficulty: number): boolean {
    const prefix = "0".repeat(difficulty);
    return hash.startsWith(prefix);
  }

  public static async solve(challenge: PowChallenge): Promise<number> {
    if (challenge.difficulty === 0 || challenge.difficulty > 32) {
      throw new Error("Invalid difficulty");
    }

    let solution = 0;
    while (true) {
      const hash = await this.hashSolution(challenge, solution);
      if (this.isSolution(hash, challenge.difficulty)) {
        return solution;
      }

      solution++;
    }
  }
}
