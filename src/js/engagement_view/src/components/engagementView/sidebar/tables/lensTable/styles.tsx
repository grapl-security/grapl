import { makeStyles } from "@material-ui/core/styles";


export const useStyles = makeStyles({
	root: {
		fontSize: "1rem",
		border: 0,
		color: "white",
		padding: "0 30px",
    },
    tableRow: {
        background: "#323232",
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
		minWidth: 400,
	},
	lensName: {
		fontSize: ".75rem",
	},
	pagination: {
		backgroundColor: "#323232"
	},
	tableHead: {
		display: "flex",
		backgroundColor: "#323232",
		color: "white",
        fontSize: ".8rem",
		minWidth: 450,
	},
	hdrTitle: {
		margin:".5rem",
		marginLeft: ".5rem"
    },
    riskTitle: {
        margin:".5rem",
        marginLeft: "10rem",
    },
    tableContainer:{
        textAlign: "center",
        marginLeft: "auto",
        marginRight: "auto",
		width: "95%"
		
    },
});
