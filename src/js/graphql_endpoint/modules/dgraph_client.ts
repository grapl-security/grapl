import * as dgraph from "dgraph-js";
import * as grpc from "@grpc/grpc-js";

const get_random = <T>(list: T[]): T => {
    return list[Math.floor(Math.random() * list.length)];
};


export type DgraphClient = dgraph.DgraphClient;
export const getDgraphClient = (): dgraph.DgraphClient => {
    return null as any // todo: fixme
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
