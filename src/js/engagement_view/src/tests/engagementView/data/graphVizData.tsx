export const vizGraphData = [
	{
		uid: 10005,
		node_key: "3db5c6b8-1c4e-414f-9947-7a02f07156fd",
		dgraph_type: ["Process"],
		process_name: "cmd.exe",
		process_id: 5824,
		children: [
			{
				uid: 10032,
				node_key: "69c90caa-cbc6-4b80-8f29-60f501aeb114",
				dgraph_type: ["Process"],
				process_name: "svchost.exe",
				process_id: 6132,
			},
		],
		risks: [
			{
				uid: 10133,
				dgraph_type: ["Risk"],
				node_key: "Suspicious svchost",
				analyzer_name: "Suspicious svchost",
				risk_score: 75,
			},
			{
				uid: 10138,
				dgraph_type: ["Risk"],
				node_key: "Rare Parent of cmd.exe",
				analyzer_name: "Rare Parent of cmd.exe",
				risk_score: 10,
			},
		],
	},
	{
		uid: 10013,
		node_key: "DESKTOP-FVSHABR",
		dgraph_type: ["Asset"],
		hostname: "DESKTOP-FVSHABR",
		asset_ip: null,
		asset_processes: [
			{
				uid: 10005,
				node_key: "3db5c6b8-1c4e-414f-9947-7a02f07156fd",
				dgraph_type: ["Process"],
				process_name: "cmd.exe",
				process_id: 5824,
			},
			{
				uid: 10030,
				node_key: "d7f7d377-1c7c-486f-9956-6ef996c5b606",
				dgraph_type: ["Process"],
				process_name: "dropper.exe",
				process_id: 4164,
			},
			{
				uid: 10032,
				node_key: "69c90caa-cbc6-4b80-8f29-60f501aeb114",
				dgraph_type: ["Process"],
				process_name: "svchost.exe",
				process_id: 6132,
			},
		],
		files_on_asset: null,
		risks: [
			{
				uid: 10133,
				dgraph_type: ["Risk"],
				node_key: "Suspicious svchost",
				analyzer_name: "Suspicious svchost",
				risk_score: 75,
			},
			{
				uid: 10138,
				dgraph_type: ["Risk"],
				node_key: "Rare Parent of cmd.exe",
				analyzer_name: "Rare Parent of cmd.exe",
				risk_score: 10,
			},
		],
	},
	{
		uid: 10030,
		node_key: "d7f7d377-1c7c-486f-9956-6ef996c5b606",
		dgraph_type: ["Process"],
		process_name: "dropper.exe",
		process_id: 4164,
		children: [
			{
				uid: 10005,
				node_key: "3db5c6b8-1c4e-414f-9947-7a02f07156fd",
				dgraph_type: ["Process"],
				process_name: "cmd.exe",
				process_id: 5824,
			},
		],
		risks: [
			{
				uid: 10138,
				dgraph_type: ["Risk"],
				node_key: "Rare Parent of cmd.exe",
				analyzer_name: "Rare Parent of cmd.exe",
				risk_score: 10,
			},
		],
	},
	{
		uid: 10032,
		node_key: "69c90caa-cbc6-4b80-8f29-60f501aeb114",
		dgraph_type: ["Process"],
		process_name: "svchost.exe",
		process_id: 6132,
		children: null,
		risks: null,
	},
];


export const vizGraphReturnData = {
	nodes: [{"name":10005,"uid":10005,"node_key":"3db5c6b8-1c4e-414f-9947-7a02f07156fd","dgraph_type":["Process"],"process_name":"cmd.exe","process_id":5824,"risk_score":0,"analyzerNames":"","id":10005,"nodeType":"Process","nodeLabel":"cmd.exe"},{"name":10032,"uid":10032,"node_key":"69c90caa-cbc6-4b80-8f29-60f501aeb114","dgraph_type":["Process"],"process_name":"svchost.exe","process_id":6132,"children":null,"risks":null,"risk_score":0,"analyzerNames":"","id":10032,"nodeType":"Process","nodeLabel":"svchost.exe"},{"name":10013,"uid":10013,"node_key":"DESKTOP-FVSHABR","dgraph_type":["Asset"],"hostname":"DESKTOP-FVSHABR","asset_ip":null,"files_on_asset":null,"risk_score":85,"analyzerNames":"Suspicious svchost, Rare Parent of cmd.exe","id":10013,"nodeType":"Asset","nodeLabel":"DESKTOP-FVSHABR"},{"name":10030,"uid":10030,"node_key":"d7f7d377-1c7c-486f-9956-6ef996c5b606","dgraph_type":["Process"],"process_name":"dropper.exe","process_id":4164,"risk_score":10,"analyzerNames":"Rare Parent of cmd.exe","id":10030,"nodeType":"Process","nodeLabel":"dropper.exe"}],
	links: [{"source":10005,"name":"children","target":10032},{"source":10013,"name":"asset_processes","target":10005},{"source":10013,"name":"asset_processes","target":10030},{"source":10013,"name":"asset_processes","target":10032},{"source":10030,"name":"children","target":10005}],
	index: {"10005":{"name":10005,"uid":10005,"node_key":"3db5c6b8-1c4e-414f-9947-7a02f07156fd","dgraph_type":["Process"],"process_name":"cmd.exe","process_id":5824,"risk_score":0,"analyzerNames":"","id":10005,"nodeType":"Process","nodeLabel":"cmd.exe"},"10013":{"name":10013,"uid":10013,"node_key":"DESKTOP-FVSHABR","dgraph_type":["Asset"],"hostname":"DESKTOP-FVSHABR","asset_ip":null,"files_on_asset":null,"risk_score":85,"analyzerNames":"Suspicious svchost, Rare Parent of cmd.exe","id":10013,"nodeType":"Asset","nodeLabel":"DESKTOP-FVSHABR"},"10030":{"name":10030,"uid":10030,"node_key":"d7f7d377-1c7c-486f-9956-6ef996c5b606","dgraph_type":["Process"],"process_name":"dropper.exe","process_id":4164,"risk_score":10,"analyzerNames":"Rare Parent of cmd.exe","id":10030,"nodeType":"Process","nodeLabel":"dropper.exe"},"10032":{"name":10032,"uid":10032,"node_key":"69c90caa-cbc6-4b80-8f29-60f501aeb114","dgraph_type":["Process"],"process_name":"svchost.exe","process_id":6132,"children":null,"risks":null,"risk_score":0,"analyzerNames":"","id":10032,"nodeType":"Process","nodeLabel":"svchost.exe"}}
}