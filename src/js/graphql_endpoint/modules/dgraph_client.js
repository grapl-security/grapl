const dgraph = require("dgraph-js");
const grpc = require("@grpc/grpc-js");

const get_random = (list) => {
	return list[Math.floor(Math.random() * list.length)];
};

const mg_alpha = get_random(process.env.MG_ALPHAS.split(","));

const getDgraphClient = () => {
	const clientStub = new dgraph.DgraphClientStub(
		mg_alpha,
		grpc.credentials.createInsecure()
	);

	return new dgraph.DgraphClient(clientStub);
};

module.exports = {
    getDgraphClient
};