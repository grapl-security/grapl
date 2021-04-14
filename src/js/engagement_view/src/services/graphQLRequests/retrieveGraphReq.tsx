import { Node } from "../../types/CustomTypes";
import { unpackPluginNodes } from "./utils_retrieveGraph/unpackPluginNodes";
import { expandLensScopeQuery } from "./utils_retrieveGraph/expandLensScopeQuery";

import DEV_API_EDGES from "../constants";
import { apiPostRequestWithBody } from "../fetch";

export const retrieveGraph = async (lens: string): Promise<Node[]> => {
    const expandScopeQueryData = expandLensScopeQuery(lens);

    const lensScopeQuery = JSON.stringify({ query: expandScopeQueryData });

    const queryResponse = await apiPostRequestWithBody(
        `${DEV_API_EDGES.graphQL}/graphql`,
        lensScopeQuery
    )
        .then((res) => res)
        .then((res) => {
            if (res.errors) {
                console.log("Unable to retrieve graph data ", res.errors);
            }
            return res;
        })
        .then((res) => res.data)
        .then((res) => res.lens_scope.scope);

    const lensWithScopeData = await queryResponse;

    unpackPluginNodes(lensWithScopeData);

    return lensWithScopeData;
};
