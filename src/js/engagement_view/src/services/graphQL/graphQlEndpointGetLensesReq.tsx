import DEV_API_EDGES from '../constants';
import {apiFetchWithBody} from '../fetch';

export const getLenses = async (first: number, offset: number) => {
    const gqlQuery = `
        {
            lenses(first: ${first}, offset: ${offset}) {
                uid,
                node_key,
                lens_name,
                score, 
                lens_type,
            }
        }
    `;

    const graphQLQuery = JSON.stringify({ query: gqlQuery })
    
    const response = 
        await apiFetchWithBody(`${DEV_API_EDGES.graphQL}/graphQlEndpoint/graphql`, "POST", graphQLQuery)
            .then(res => res)
            .then(res => {
                if (res.errors) {
                    console.error("Unable to retrieve lenses ", res.errors);
                    res.data = {lenses: []};
                }
                return res
            })
            .then((res) => res.data);

    const lensQueryData = await response;
    console.log("Retrieved lenses: ", lensQueryData);
    
    return lensQueryData;
};
