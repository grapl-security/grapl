import { getNodeLabel } from "components/graphDisplay/graphLayout/labels";

import { processNode } from "./engagementView/testData/baseData";

// Test graph styling for high risk process node
// Assuming if process works, formatting for other node types also works

test("get node label", () => {
    expect(getNodeLabel("Process", processNode as any)).toBe("Process");
});
