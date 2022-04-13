import {
    getLinkLabel,
    getNodeLabel,
} from "components/graphDisplay/graphLayout/labels";

import { processNode } from "./engagementView/testData/baseData";

test("link label for children", () => {
    expect(getLinkLabel("children")).toBe("children");
});

// Test graph styling for high risk process node
// Assuming if process works, formatting for other node types also works
test("link label for processes", () => {
    expect(getLinkLabel("asset_processes")).toBe("asset_processes");
});

test("get node label", () => {
    expect(getNodeLabel("Process", processNode as any)).toBe("Process");
});
