import { useEffect, useState } from 'react';

// export default SubnetStatus;
const DashboardPage: React.FC = () => {
	const [networks, setNetworks] = useState<string[]>([]);
	const [selectedNetwork, setSelectedNetwork] = useState<string | null>(null);

	const fetchData = async () => {
		// Simulate an API request to get the list of potential networks
		// Replace this with your actual API request
		const data = ["a", "b", "c", "d", "e", "f"];
		setNetworks(data);
	};

	const handleNetworkSelect = (network: string) => {
		setSelectedNetwork(network);
	};

	useEffect(() => {
		// Fetch data when the component mounts
		fetchData();

		// Set up an interval to refresh the data every 10 seconds
		const intervalId = setInterval(fetchData, 10000);

		// Clean up the interval on component unmount
		return () => clearInterval(intervalId);
	}, []); // Run the effect only once when the component mounts

	return (
		<div className="flex h-screen bg-gray-100">
			{/* Sidebar with Dynamic Links */}
			<DashboardSidebar networks={networks} onNetworkSelect={handleNetworkSelect} />

			{/* Main Content */}
			<main className="flex-1 p-4 overflow-hidden">
				{/* Data Display based on the selected network */}
				{selectedNetwork && (
					<DataDisplay network={selectedNetwork} />
				)}
			</main>
		</div>
	);
};

const DashboardSidebar: React.FC<{ networks: string[]; onNetworkSelect: (network: string) => void }> = ({ networks, onNetworkSelect }) => {
	return (
		<aside className="w-64 bg-blue-500 p-4">
			<h1 className="text-white text-2xl font-semibold">Data Dashboard</h1>
			<nav className="mt-4">
				<ul>
					{networks.map(network => (
						<li key={network} className="mb-2">
							<a
								href={`#${network}`}
								className="text-white hover:text-gray-300"
								onClick={() => onNetworkSelect(network)}
							>
								{network}
							</a>
						</li>
					))}
				</ul>
			</nav>
		</aside>
	);
};

const DataDisplay: React.FC<{ network: string }> = ({ network }) => {
	// Fetch data for the selected network
	// You can make another API request here or use local state to manage the data

	return (
		<main className="flex-1 p-4 overflow-hidden">
			{/* Page Title */}
			<div className="mb-4">
				<h2 className="text-2xl font-semibold">{network}</h2>
			</div>

			{/* Cards */}
			<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
				{/* Card 1 */}
				<div className="bg-white p-4 rounded-md shadow-md">
					<h3 className="text-xl font-semibold mb-2">Total Users</h3>
					<p className="text-gray-600">1,234</p>
				</div>

				{/* Card 2 */}
				<div className="bg-white p-4 rounded-md shadow-md">
					<h3 className="text-xl font-semibold mb-2">Revenue</h3>
					<p className="text-gray-600">$56,789</p>
				</div>

				{/* Card 3 */}
				<div className="bg-white p-4 rounded-md shadow-md">
					<h3 className="text-xl font-semibold mb-2">New Orders</h3>
					<p className="text-gray-600">456</p>
				</div>

				{/* Card 4 */}
				<div className="bg-white p-4 rounded-md shadow-md">
					<h3 className="text-xl font-semibold mb-2">Active Users</h3>
					<p className="text-gray-600">789</p>
				</div>
			</div>

			{/* Chart */}
			<div className="mt-8">
				{/* Replace the following div with your chart component */}
				<div className="bg-white p-4 rounded-md shadow-md">Your Chart Goes Here</div>
			</div>
		</main>
	);
};

export default DashboardPage;