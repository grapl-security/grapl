import {
    GraphQLField,
    GraphQLList,
    GraphQLObjectType,
    GraphQLUnionType,
} from "graphql";

const DEFAULT_STACK_LIMIT = 2; // otherwise you'd have recursion while expanding all the types
const TABBING = 4; // size of a tab, in number of spaces

export class QueryGenerator {
    constructor(readonly GraplEntityType: GraphQLUnionType) {}

    genField(args: {
        field: GraphQLField<any, any>;
        stackLimit: number;
        numSpaces: number;
    }): string[] {
        /**
         * Sample outputs:
         * hostname
         * asset_ip{
             ip_address
           }
         **/
        if (args.stackLimit == 0) {
            return [];
        }
        const spaces = " ".repeat(args.numSpaces);
        const type = args.field.type;
        const children: string[] = [];
        if (type instanceof GraphQLObjectType) {
            const childrenFields = type.getFields();
            for (const key in childrenFields) {
                children.push(
                    ...this.genField({
                        field: childrenFields[key],
                        stackLimit: args.stackLimit - 1,
                        numSpaces: args.numSpaces + TABBING,
                    })
                );
            }
        } else if (
            type instanceof GraphQLList &&
            type.ofType instanceof GraphQLObjectType
        ) {
            const childrenFields = type.ofType.getFields();
            for (const key in childrenFields) {
                children.push(
                    ...this.genField({
                        field: childrenFields[key],
                        stackLimit: args.stackLimit - 1,
                        numSpaces: args.numSpaces + TABBING,
                    })
                );
            }
        } else {
            // it's a scalar, which we can handle easily
            return [`${spaces}${args.field.name}`];
        }
        // it's an object type
        if (children.length) {
            return [
                `${spaces}${args.field.name} {
${children.join(",\n")}
${spaces}}`,
            ];
        }
        // it's an object type, but we don't query any predicates on it - so just elide it
        // for example, `risks { }` -> just return nothing
        return [];
    }

    genOnFragment(args: {
        type: GraphQLObjectType;
        numSpaces: number;
    }): string {
        const typeName = args.type.name;
        const fields: string[] = [];
        for (const key in args.type.getFields()) {
            const field = args.type.getFields()[key];
            fields.push(
                ...this.genField({
                    field: field,
                    stackLimit: DEFAULT_STACK_LIMIT,
                    numSpaces: args.numSpaces + TABBING,
                })
            );
        }
        const spaces = " ".repeat(args.numSpaces);
        return `\n${spaces}... on ${typeName} {
${fields.join(",\n")}
${spaces}}`;
    }

    public generate(): string {
        const scopeDefinition = this.GraplEntityType.getTypes().map((type) =>
            this.genOnFragment({ type: type, numSpaces: TABBING * 3 })
        );
        return `
query LensScopeByName($lens_name: String!) {
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
}
        `;
    }
}
