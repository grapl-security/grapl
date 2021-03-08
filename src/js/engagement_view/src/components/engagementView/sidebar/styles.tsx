import { makeStyles } from "@material-ui/core/styles";


export const useStyles = makeStyles({
	root: {
		fontSize: "1rem",
		border: 0,
		color: "white",
		padding: "0 30px",
    },
    tableRow: {
        background: "#333333"
    },
	backdrop: {
		color: "#fff",
		backgroundColor: "transparent",
		width: "80%",
	},
	button: {
		width: ".05%",
		height: "50%",
		color: "white",
		margin: ".5rem",
		backgroundColor: "#424242",
	},
	title: {
		margin: "1rem",
        fontSize: "1.1rem",
        color: "white",
	},
	expand: {
		color: "#42C6FF",
		margin: "0px",
		width: "1.5rem",
		height: "1.5rem",
	},
	header: {
		display: "flex",
	},
	table: {
		minWidth: 450,
		backgroundColor: "#595959",
	},
	lensName: {
		fontSize: ".75rem",
	},
	pagination: {
		margin: ".5rem",
		backgroundColor: "#323232",
	},
	head: {
		display: "flex",
		backgroundColor: "#363434",
		color: "white",
        fontSize: ".75rem",
        padding: "1em"
	},
	hdrTitle: {
        fontSize: "1rem",
        margin:".5rem",
        color: "#fff",
    },
    riskTitle: {
        fontSize: "1rem",
        margin:".5rem",
        marginLeft: "10rem",
        color: "#fff",
    },
    tableContainer:{
        textAlign: "center",
        marginLeft: "auto",
        marginRight: "auto",
        width: "95%"
    },
});
