import dgraph = require("dgraph-js");
import grpc = require("@grpc/grpc-js");

const get_random = <T>(list: T[]): T => {
	return list[Math.floor(Math.random() * list.length)];
};

const mg_alpha = get_random(process.env.MG_ALPHAS.split(","));

export const getDgraphClient = (): dgraph.DgraphClient => {
	const clientStub = new dgraph.DgraphClientStub(
		mg_alpha,
		grpc.credentials.createInsecure()
	);

	return new dgraph.DgraphClient(clientStub);
};