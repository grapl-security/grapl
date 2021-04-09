import * as dgraph from "dgraph-js";
import * as grpc from "@grpc/grpc-js";

const get_random = <T>(list: T[]): T => {
    return list[Math.floor(Math.random() * list.length)];
};

const mg_alpha = get_random(process.env.MG_ALPHAS.split(","));

export type DgraphClient = dgraph.DgraphClient;
export const getDgraphClient = (): dgraph.DgraphClient => {
    const clientStub = new dgraph.DgraphClientStub(
        mg_alpha,
        grpc.credentials.createInsecure()
    );

    return new dgraph.DgraphClient(clientStub);
};

export interface RawNode {
  uid: number | string;
  dgraph_type?: string[];
}

export interface EnrichedNode {
    readonly uid: number;
    dgraph_type: string[] | undefined;
    [predicate: string]: any;
    display: string;
}
