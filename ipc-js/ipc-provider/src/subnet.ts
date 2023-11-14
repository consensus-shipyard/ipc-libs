import { DelegatedAddress } from "./address";

export class SubnetID {
	root: number;
	children: DelegatedAddress[];

	constructor(root: number, children: DelegatedAddress[]) {
		this.root = root;
		this.children = children;
	}

	static newFromParent(parent: SubnetID, subnetAct: DelegatedAddress): SubnetID {
		const children = parent.children.slice();
		children.push(subnetAct);
		return new SubnetID(parent.root, children);
	}

	static newRoot(rootId: number): SubnetID {
		return new SubnetID(rootId, []);
	}

	isRoot(): boolean {
		return this.children.length === 0;
	}

	toString(): string {
		let result = 'r' + this.root;

		for (const child of this.children) {
			result += '/' + child;
		}

		return result;
	}

	rootId(): number {
		return this.root;
	}

	subnetActor(): DelegatedAddress | undefined {
		if (this.children.length > 0) {
			return this.children[this.children.length - 1];
		} else {
			throw new Error("Root subnet has no subnet actor");
		}
	}

	subnetActorEthAddr(): string {
		const subnetActor = this.subnetActor()?.ethereumAddress();
		if (subnetActor === undefined) {
			throw new Error("Subnet Actor is undefined");
		}
		return subnetActor;
	}

	parent(): SubnetID | null {
		if (this.children.length === 0) {
			return null;
		}

		const children = this.children.slice();
		children.pop();
		return new SubnetID(this.root, children);
	}

	equals(other: any): boolean {
		console.log("asdfsadf");
		if (this === other) {
			console.log(1);
			return true;
		}

		if (!(other instanceof SubnetID)) {
			console.log(1);
			return false;
		}

		if (this.root !== other.root) {
			console.log(2);
			return false;
		}

		if (this.children.length !== other.children.length) {
			console.log(3);
			return false;
		}

		for (let i = 0; i < this.children.length; i++) {
			console.log(4);
			if (!this.children[i]?.equals(other.children[i])) {
				return false;
			}
		}

		return true;
	}

}