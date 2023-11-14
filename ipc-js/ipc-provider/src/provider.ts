import { ethers } from 'ethers';
import { Config, newConfigForNetwork } from './config';
import { importABIs } from './config';
import log from 'loglevel';

import { Contract, Wallet, JsonRpcProvider } from 'ethers';
import { SubnetID } from './subnet';

// Import IPC ABIs to make it available throughout the provider.
const contractABIs = importABIs();

class Connection {
	provider: JsonRpcProvider;
	gateway: string;

	constructor(provider: JsonRpcProvider, gateway: string) {
		this.provider = provider;
		this.gateway = gateway;
	}

	getContract(contractName: string, address: string): Contract {
		const abi = contractABIs.get(contractName);

		if (!abi) {
			throw new Error(`ABI for contract ${contractName} not found.`);
		}

		const contract = new Contract(address, abi, this.provider);
		if (contract) {
			return contract
		}

		throw new Error("contract not found")
	}

	getGatewayFacet(facetName: string): Contract {
		return this.getContract(facetName, this.gateway)
	}
}

export class IpcProvider {
	private config: Config;

	constructor(
		config: Config,
	) {
		this.config = config;
	}

	static newForNetwork(network: string): IpcProvider {
		return new IpcProvider(newConfigForNetwork(network));
	}


	connection(subnet: SubnetID): Connection {
		const endpoint = this.config.subnets.get(subnet.toString())?.fevm.provider_http;
		if (endpoint) {
			const gateway = this.config.subnets.get(subnet.toString())?.fevm.gateway_addr;
			if (gateway === undefined) {
				throw new Error("Gateway is undefined");
			}
			return new Connection(
				new JsonRpcProvider(endpoint),
				gateway,
			);
		}
		throw new Error("Subnet not configured in provider: " + subnet);
	}

	async listSubnets(subnet: SubnetID) {
		let contract = this.connection(subnet).getGatewayFacet("Gateway")
		// Try manually
		console.log(await contract.listSubnets())
	}
}
