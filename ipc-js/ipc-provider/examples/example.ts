import { IpcProvider } from '../src/provider';
import { SubnetID } from '../src/subnet';


let provider = IpcProvider.newForNetwork("calibration");
console.log(SubnetID.newRoot(123));
let conn = provider.connection(SubnetID.newRoot(314159));
provider.listSubnets(SubnetID.newRoot(314159));
