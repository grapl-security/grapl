import { getGraphQlEdge } from "../services/getApiURLs";

const graphql_edge = getGraphQlEdge();

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

    console.log("calling graphql_edge: " + graphql_edge + "with query: " + gqlQuery);
    
    const res = await fetch(`${graphql_edge}graphQlEndpoint/graphql`,
        {
            method: 'post',
            body: JSON.stringify({ query: gqlQuery }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        })
        .then(res => res.json())
        .then(res => {
            if (res.errors) {
                console.error("lenses failed", res.errors);
                res.data = {lenses: []};
            }
            return res
        })
        .then((res) => res.data);

        const jres = await res;

        console.log("queried graphql_edge in engagement view content", jres);
    return jres;
};
