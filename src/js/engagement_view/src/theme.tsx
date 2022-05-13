import { createTheme } from "@mui/material/styles";

const theme = createTheme({
    palette: {
        mode: "dark",
        primary: {
            main: "#2196f3",
            contrastText: "#56657F",
            light: "#AFBDD1",
        },
        secondary: {
            main: "#c65454",
            light: "#AFBDD1",
            contrastText: "#2B3648",
        },
        background: {
            default: "#212936",
            paper: "#1976D2", // paper highlight
        },
        text: {
            primary: "#d9e5fc",
            secondary: "#AFBDD1",
        },
        success: {
            main: "#7cd27a",
            contrastText: "#FFFFFF",
        },
        info: {
            main: "#2196f3",
            light: "#65b5f6",
            contrastText: "#FFFFFF",
        },
    },
    components: {
        MuiListItemButton: {
            styleOverrides: {
                root: {
                    "&.Mui-selected": {
                        color: "#1976D2",
                        background: "#2B3648",
                    },
                },
            },
        },
    },
});

export default theme;
