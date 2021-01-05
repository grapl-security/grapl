import {BaseNode, LensScopeResponse} from '../../types/CustomTypes';
import {unpackPluginNodes} from './utils_GraphQlEndpointRetrieveGraph/unpackPluginNodes';
import {expandScopeQuery} from './utils_GraphQlEndpointRetrieveGraph/expandScopeQuery';

import DEV_API_EDGES from '../constants';
import {apiFetchPostRequest} from '../fetch';

export const retrieveGraph = async (lens: string): Promise<(LensScopeResponse & BaseNode)> => {
    const expandScopeQueryData = expandScopeQuery(lens);

    const lensScopeQuery = JSON.stringify({ query: expandScopeQueryData })

    const queryResponse = 
        await apiFetchPostRequest(`${DEV_API_EDGES.graphQL}/graphQlEndpoint/graphql`, "POST", lensScopeQuery)
            .then(res => res)
            .then(res => {
                if(res.errors){
                    console.log("Unable to retrieve graph data ", res.errors)
                }
                console.log('Retrieved Graph Data: ', res);
                return res
            })
            .then((res) => res.data)
            .then((res) => res.lens_scope);

    const lensWithScopeData = await queryResponse;
    console.debug('LensWithScope: ', lensWithScopeData);

    unpackPluginNodes(lensWithScopeData.scope);

    return lensWithScopeData;
};
