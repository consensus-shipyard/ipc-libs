/* global ethers */
/* eslint prefer-const: "off" */
import { Contract, Wallet, JsonRpcProvider } from 'ethers';

const PRIVATE_KEY = "0x0123456789012345678901234567890123456789012345678901234567890123";
const RPC_URL = "https://api.calibration.node.glif.io/rpc/v1";
const GATEWAY_ADDRESS = '0x5Be26735Ab7A70B057e76e31953d8811f95b05AC';
import { Contract, Wallet, JsonRpcProvider } from 'ethers';
import path from 'path';
import fs from 'fs';

function readAbiFromDirectory(directory: string): any[] {
    const abiPath = path.resolve(directory, 'gatewayAbi.json');
    const abi = JSON.parse(fs.readFileSync(abiPath, 'utf8')) as any[];
    return abi;
}

// Example usage
const directory = path.resolve(__dirname, '..', 'ipc-solidity');
const gatewayAbi = readAbiFromDirectory(directory);


async function connectGateway(gatewayAddress: string, abi: any[], privateKey: string, rpcUrl: string) {
    let provider = new JsonRpcProvider(rpcUrl);
    let signer = new Wallet(privateKey, provider);

    console.log("block number: ", await provider.getBlockNumber());

    const contract = new Contract(gatewayAddress, abi, signer);

    const parent = { route: [], root: "r314159" };
    const subnetId = { route: [], root: "r314159" };

    console.log("subnets: ", await contract.listSubnets());
    // console.log("parent finality", await contract.getParentFinality(1000));
    // console.log("subnet: ", await contract.getSubnet({ route: [], root: "r314159" }));
    // console.log("top down messages: ", await contract.getTopDownMsgs({ route: [], root: "r314159" }, 10));
}

connectGateway(GATEWAY_ADDRESS, GATEWAY_ABI, PRIVATE_KEY, RPC_URL);
