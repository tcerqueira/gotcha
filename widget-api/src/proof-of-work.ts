export interface PowChallenge {
  nonce: number;
  difficulty: number;
  timestamp: number;
}

export class ProofOfWork {
  private static readonly HASH_ALGORITHM = "SHA-256";

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

  private static isSolution(hash: string, difficulty: number): boolean {
    const prefix = "0".repeat(difficulty);
    return hash.startsWith(prefix);
  }

  private static async hashSolution(
    challenge: PowChallenge,
    solution: number,
  ): Promise<string> {
    const nonce_bytes = this.toBeBytes(challenge.nonce, 4);
    const difficulty_bytes = this.toBeBytes(challenge.difficulty, 2);
    const timestamp_bytes = this.toBeBytes(challenge.timestamp, 8, true);
    const solution_bytes = this.toBeBytes(solution, 4);

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

  private static toBeBytes(
    num: number | bigint,
    byteSize: number,
    signed: boolean = false,
  ): Uint8Array {
    const buffer = new ArrayBuffer(byteSize);
    const view = new DataView(buffer);

    switch (byteSize) {
      case 1:
        signed ? view.setInt8(0, Number(num)) : view.setUint8(0, Number(num));
        break;
      case 2:
        signed
          ? view.setInt16(0, Number(num), false)
          : view.setUint16(0, Number(num), false);
        break;
      case 4:
        signed
          ? view.setInt32(0, Number(num), false)
          : view.setUint32(0, Number(num), false);
        break;
      case 8:
        signed
          ? view.setBigInt64(0, BigInt(num), false)
          : view.setBigUint64(0, BigInt(num), false);
        break;
      default:
        throw new Error(`Unsupported byte size: ${byteSize}`);
    }

    return new Uint8Array(buffer);
  }
}
