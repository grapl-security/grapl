import {BaseNode, LensScopeResponse} from '../../types/CustomTypes';
import {getGraphQlEdge} from '../getApiURLs';
import {unpackPluginNodes} from './utils_GraphQlEndpointRetrieveGraph/unpackPluginNodes';
import {expandScopeQuery} from './utils_GraphQlEndpointRetrieveGraph/expandScopeQuery'

const graphql_edge = getGraphQlEdge();

export const retrieveGraph = async (lens: string): Promise<(LensScopeResponse & BaseNode)> => {
    const query = expandScopeQuery(lens);
    
    console.log("in retreive graph calling graphql edge", graphql_edge);

    const res = await fetch(`${graphql_edge}graphQlEndpoint/graphql`,
        {
            method: 'post',
            body: JSON.stringify({ query }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        })
        .then(res => res.json())
        .then(res => {
            if(res.errors){
                console.log("graphql query failed in retrieve graph, expand scope ln 63: ", res.errors)
            }
            console.log('retrieveGraph res', res);
            return res
        })
        .then((res) => res.data)
        .then((res) => res.lens_scope);

    const lensWithScope = await res;

    console.debug('LensWithScope: ', lensWithScope);

    unpackPluginNodes(lensWithScope.scope);

    return lensWithScope;
};
