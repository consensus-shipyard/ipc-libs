import { ethers } from 'ethers';

/// Max length of f4 sub addresses.
const MAX_SUBADDRESS_LEN: number = 54;
type ActorID = number;

export class DelegatedAddress {
	namespace: ActorID;
	length: number;
	buffer: Uint8Array;

	constructor(namespace: ActorID, subAddress: Uint8Array) {
		const length = subAddress.length;
		if (length > MAX_SUBADDRESS_LEN) {
			throw new Error(`InvalidPayloadLength: ${length}`);
		}
		this.namespace = namespace;
		this.length = length;
		this.buffer = new Uint8Array(MAX_SUBADDRESS_LEN);
		this.buffer.set(subAddress.slice(0, length));
	}

	ethereumAddress(): string {
		return ethers.hexlify(this.buffer);
	}

	equals(other: any): boolean {
		if (this === other) {
			return true;
		}

		if (!(other instanceof DelegatedAddress)) {
			return false;
		}

		if (this.namespace !== other.namespace) {
			return false;
		}

		if (this.length !== other.length) {
			return false;
		}

		if (this.buffer.length !== other.buffer.length) {
			return false;
		}

		for (let i = 0; i < this.buffer.length; i++) {
			if (this.buffer[i] !== other.buffer[i]) {
				return false;
			}
		}

		return true;
	}
}
