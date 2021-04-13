import {
    GraphQLField,
    GraphQLFieldMap,
    GraphQLList,
    GraphQLObjectType,
    GraphQLOutputType,
    GraphQLType,
    GraphQLUnionType,
} from "graphql";

const DEFAULT_STACK_LIMIT = 2; // otherwise you'd have recursion while expanding all the types

export class QueryGenerator {
    constructor(readonly GraplEntityType: GraphQLUnionType) {}

    genField(
        field: GraphQLField<any, any>,
        stackLimit: number
    ): string[] {
        /**
         * Sample outputs:
         * hostname
         * asset_ip{
             ip_address
           }
         **/
        if(stackLimit == 0) {
            return [];
        }
        const children: string[] = [];
        if (field.type instanceof GraphQLObjectType) {
            const childrenFields = field.type.getFields();
            for (const key in childrenFields) {
                children.push(...this.genField(childrenFields[key], stackLimit - 1));
            }
        } else if (
            field.type instanceof GraphQLList &&
            field.type.ofType instanceof GraphQLObjectType
        ) {
            const childrenFields = field.type.ofType.getFields();
            for (const key in childrenFields) {
                children.push(
                    ...
                    this.genField(childrenFields[key], stackLimit - 1)
                );
            }
        } else {
            // it's a scalar, which we can handle easily
            return [field.name];
        }
        // it's an object type
        if (children.length) {
            return [`${field.name} {${children.join(",")}}`]
        }
        // it's an object type, but we don't query any predicates on it - so just elide it
        // for example, `risks { }` -> just return nothing
        return []; 
    }

    genOnFragment(type: GraphQLObjectType): string {
        const typeName = type.name;
        const fields: string[] = [];
        for (const key in type.getFields()) {
            const field = type.getFields()[key];
            fields.push(...this.genField(field, DEFAULT_STACK_LIMIT));
        }
        return ` ... on ${typeName} {
            ${fields.join(",")}
        }`;
    }

    public generate(): string {
        const scopeDefinition = this.GraplEntityType.getTypes().map((type) =>
            this.genOnFragment(type)
        );
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
        `;
