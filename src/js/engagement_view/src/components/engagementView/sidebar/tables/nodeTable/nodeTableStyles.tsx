import { makeStyles } from "@material-ui/core/styles";
import theme from "../../../../../theme";

export const nodeTableStyles = makeStyles({
    root: {
        fontSize: ".75em",
    },
    nodeTable: {
        minWidth: 450,
    },
    tableHeader: {
        fontSize: ".85rem",
        backgroundColor: theme.palette.background.default,
    },
    nodeTableData: {
        fontSize: ".75rem",
        backgroundColor: theme.palette.background.default,
    },
    nodeTableContainer: {
        textAlign: "center",
        marginLeft: "auto",
        marginRight: "auto",
        width: "95%",
    },
});
