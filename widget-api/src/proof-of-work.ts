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
    const nonce_bytes = new Uint8Array(
      new Uint32Array([challenge.nonce]).buffer,
    );
    const difficulty_bytes = new Uint8Array(
      new Uint16Array([challenge.difficulty]).buffer,
    );
    const timestamp_bytes = new Uint8Array(
      new BigInt64Array([BigInt(challenge.timestamp)]).buffer,
    );
    const solution_bytes = new Uint8Array(new Uint32Array([solution]).buffer);

    const data = new Uint8Array([
      ...nonce_bytes,
      ...difficulty_bytes,
      ...timestamp_bytes,
      ...solution_bytes,
    ]);

    const hashBuffer = await crypto.subtle.digest(this.HASH_ALGORITHM, data);
    const hash = Array.from(new Uint8Array(hashBuffer))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return hash;
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
