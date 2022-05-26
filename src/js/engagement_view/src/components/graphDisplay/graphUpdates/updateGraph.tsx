import { GraphState } from "../../../types/GraphDisplayTypes";

import { retrieveGraph } from "../../../services/graphQLRequests/retrieveGraphReq";
import { vizGraphFromLensScope } from "../graphLayout/vizGraphFromLensScope";
import { mergeGraphs } from "../graphLayout/mergeGraphs";

export const updateGraph = async (
    lensName: string,
    engagementState: GraphState,
    setEngagementState: (engagementState: GraphState) => void
) => {
    if (!lensName) {
        console.log("No lenses available");
        return;
    }

    const curLensName = engagementState.curLensName;

    await retrieveGraph(lensName)
        .then(async (scope) => {
            const updatedGraph = vizGraphFromLensScope(scope);

            const mergeUpdatedGraph = mergeGraphs(
                engagementState.graphData,
                updatedGraph
            );

            if (mergeUpdatedGraph !== null) {
                if (curLensName === lensName) {
                    setEngagementState({
                        ...engagementState,
                        curLensName: lensName,
                        graphData: mergeUpdatedGraph,
                    });
                } else {
                    setEngagementState({
                        ...engagementState,
                        curLensName: lensName,
                        graphData: updatedGraph,
                    });
                }
            }
        })
        .catch((e) => console.error("Failed to retrieveGraph ", e));
};
