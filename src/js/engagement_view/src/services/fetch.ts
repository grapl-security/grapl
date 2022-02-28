import { Node } from "types/CustomTypes";
import DEV_API_EDGES from "./constants";

export const apiFetchReq = async (urlSlug: string, method = "GET") => {
    const response = await fetch(urlSlug, {
        method,
        credentials: "include",
        headers: new Headers({
            "Content-Type": "application/json",
            // Enable Consul Ingress Gateway tracing with custom header
            "x-client-trace-id": "1",
        }),
    }).catch((e) => {
        console.warn(e);
        throw new Error(`API Request Error: ${e}`);
    });

    return response.json();
};

export const apiPostRequestWithBody = async (urlSlug: string, body: string) => {
    const response = await fetch(urlSlug, {
        method: "POST",
        credentials: "include",
        headers: new Headers({
            "Content-Type": "application/json",
            // Enable Consul Ingress Gateway tracing with custom header
            "x-client-trace-id": "1",
        }),
        body: body,
    }).catch((e) => {
        console.warn(e);
        throw new Error(`Error with Post Request: ${e}`);
    });

    return response.json();
};

type QueryResult = { [key: string]: any };
interface LensScope {
    lens_name: string;
    scope: Node[];
}

export class GraphqlEndpointClient {
    async query(
        query: string,
        variables?: { [key: string]: string }
    ): Promise<QueryResult> {
        const body = JSON.stringify({
            query: query,
            variables: variables || {},
        });
        const response = await apiPostRequestWithBody(
            `${DEV_API_EDGES.graphQL}/graphql`,
            body
        );
        if (response["errors"]) {
            console.log(response["errors"]);
            throw new Error(`Could not query GraphQL: ${response["errors"]}`);
        }
        return response["data"];
    }

    public async getLensScope(lensName: string): Promise<LensScope> {
        const query = await this.getScopeQuery();
        const resp = await this.query(query, { lens_name: lensName });
        return resp["lens_scope"] as LensScope;
    }

    async getScopeQuery() {
        const query = `
        {
            lens_scope_query {
                query_string
            }
        }
        `;

        const resp = await this.query(query);
        return resp["lens_scope_query"]["query_string"];
    }
}
