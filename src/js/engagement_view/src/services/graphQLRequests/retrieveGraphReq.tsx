import { Node } from "../../types/CustomTypes";

import { GraphqlEndpointClient } from "../fetch";

export const retrieveGraph = async (lens: string): Promise<Node[]> => {
    const client = new GraphqlEndpointClient();
    const lensWithScopeData = await client.getLensScope(lens);
    return lensWithScopeData.scope;
};
