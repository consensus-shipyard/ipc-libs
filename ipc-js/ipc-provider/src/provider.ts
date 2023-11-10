import { ethers } from 'ethers';

export class IpcProvider {
	private sender: any;
	private config: Config;
	private fvm_wallet?: Wallet;
	private evm_keystore?: EvmKeystore;

	constructor(
		config: Config,
		fvm_wallet?: Wallet,
		evm_keystore?: EvmKeystore
	) {
		this.sender = null;
		this.config = config;
		this.fvm_wallet = fvm_wallet;
		this.evm_keystore = evm_keystore;
	}

	static newFromConfig(configPath: string): IpcProvider {
		const config = Config.fromFile(configPath);
		const fvm_wallet = Wallet.new(
			newFvmWalletFromConfig(config)
		);
		const evm_keystore = newEvmKeystoreFromConfig(config);
		return new IpcProvider(config, fvm_wallet, evm_keystore);
	}

	static newWithSubnet(
		keystorePath: string | undefined,
		subnet: Subnet
	): IpcProvider {
		const config = new Config();
		config.addSubnet(subnet);

		if (keystorePath) {
			const fvm_wallet = Wallet.new(
				newFvmKeystoreFromPath(keystorePath)
			);
			const evm_keystore = newEvmKeystoreFromPath(keystorePath);
			return new IpcProvider(config, fvm_wallet, evm_keystore);
		} else {
			return new IpcProvider(config);
		}
	}

	static newDefault(): IpcProvider {
		const configPath = defaultConfigPath();
		return IpcProvider.newFromConfig(configPath);
	}

	connection(subnet: SubnetID): Connection | undefined {
		const subnets = this.config.subnets;
		const subnetConfig = subnets.get(subnet);
		if (subnetConfig && subnetConfig instanceof FevmSubnetConfig) {
			try {
				const wallet = this.evm_wallet();
				const manager = EthSubnetManager.from_subnet_with_wallet_store(subnetConfig, wallet);
				return new Connection(manager, subnet);
			} catch (e) {
				console.warn(`Error initializing evm wallet: ${e}`);
			}
		}
		return undefined;
	}

	private evm_wallet(): Wallet | undefined {
		if (this.fvm_wallet) {
			return this.fvm_wallet.read();
		}
		return undefined;
	}
}

// Example usage:
async function example() {
	const configPath = 'path/to/config';
	const keystorePath = 'path/to/keystore';

	const ipcProvider1 = IpcProvider.newFromConfig(configPath);
	const ipcProvider2 = IpcProvider.newWithSubnet(keystorePath, subnet);

	const connection = ipcProvider1.connection(subnet);
	if (connection) {
		const response = await connection.manager.send('eth_getBlockByNumber', ['latest', true]);
		console.log('Response:', response);
	}
}

example();