import { makeStyles } from "@material-ui/core/styles";


export const useStyles = makeStyles({
	root: {
		fontSize: "1rem",
		border: 0,
		color: "white",
		padding: "0 30px",
	},
	backdrop: {
		color: "#fff",
		backgroundColor: "transparent",
		width: "80%",
	},
	table: {
		minWidth: 450,
		backgroundColor: "#323232"
	},
	lensToggleBtn: {
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
	lensName: {
		fontSize: ".7rem",
	},
	pagination: {
		backgroundColor: "#323232"
	},
	tableHead: {
		display: "flex",
		color: "white",
        fontSize: ".8rem",
	},
    tableContainer:{
        textAlign: "center",
        marginLeft: "auto",
        marginRight: "auto",
		width: "95%"
	},
	tableRow: {
        background: "#323232",
    },
});
