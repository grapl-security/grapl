import { GraphQLFieldMap, GraphQLObjectType, GraphQLOutputType, GraphQLType, GraphQLUnionType } from "graphql";

export class QueryGenerator {
    constructor(readonly GraplEntityType: GraphQLUnionType) { 
    }

    private genField(key: string, fields: GraphQLFieldMap<any, any>): string {
        /**
         * Sample outputs:
         * hostname
         * asset_ip{
             ip_address
           }
         **/
        const field = fields[key].name;
        /*
        let childrenOfField = "";
        if ("getTypes" in field.type) {
            const children = field.type.getTypes();
        }
        */
        return `${field}`;
    }

    private genOnFragment(type: GraphQLObjectType): string {
        const typeName = type.name;
        const fields: string[] = [];
        for(const key in type.getFields()) {
            fields.push(this.genField(key, type.getFields()));
        }
        return ` ... on ${typeName} {
            ${fields.join(",")}
        }`;
    }
    
    public generate(): string {
        const scopeDefinition = this.GraplEntityType.getTypes().map((type) => this.genOnFragment(type));
        return `query LensScopeByName($lens_name: String!) {
            
            lens_scope(lens_name: $lens_name) {
                uid,
                node_key,
                lens_type,
                dgraph_type,
                score,
                display,
                scope { 
                    ${scopeDefinition.join(",")}
                }
            }
        }`;
    }
}

const example = `
        query LensScopeByName($lens_name: String!) {
            
            lens_scope(lens_name: $lens_name) {
                uid,
                node_key,
                lens_type,
                dgraph_type,
                score,
                display,
                scope {
                    ... on Process {
                        uid,
                        node_key, 
                        display,
                        dgraph_type,
                        process_name, 
                        process_id,
                        children {
                            uid, 
                            node_key, 
                            dgraph_type,
                            process_name, 
                            process_id,
                        }, 
                        risks {  
                            uid,
                            dgraph_type,
                            node_key, 
                            analyzer_name, 
                            risk_score
                        },
                    }
                    ... on Asset {
                        uid, 
                        node_key, 
                        dgraph_type,
                        display,
                        hostname,
                        asset_ip{
                            ip_address
                        }, 
                        asset_processes{
                            uid, 
                            node_key, 
                            dgraph_type,
                            process_name, 
                            process_id,
                        },
                        files_on_asset{
                            uid, 
                            node_key, 
                            dgraph_type,
                            file_path
                        }, 
                        risks {  
                            uid,
                            dgraph_type,
                            node_key, 
                            analyzer_name, 
                            risk_score
                        },
                    }
                    ... on File {
                        uid,
                        node_key, 
                        dgraph_type,
                        risks {  
                            uid,
                            dgraph_type,
                            node_key, 
                            analyzer_name, 
                            risk_score
                        },
                    }
                }
            }
        }
        `