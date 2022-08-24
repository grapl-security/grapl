import React, { SyntheticEvent } from "react";
import { styled, useTheme } from "@mui/material/styles";

import Box from "@mui/material/Box";
import Paper from "@mui/material/Paper";
import { DataGrid, GridColDef, GridCellParams } from "@mui/x-data-grid";

import { Field, Form, Formik } from "formik";
import clsx from 'clsx';

//Paper (Notifications) Icons
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorIcon from "@mui/icons-material/Error";
import NotificationsIcon from "@mui/icons-material/Notifications";

import { useStyles } from "../styles/analyzersAndGeneratorsStyles";
import "../../index.css";
import { NavigationDrawer } from "../reusableComponents/drawer";


import UploadPluginModal from "./createPluginModal";


export const AnalyzersListTable = () => {
    const classes = useStyles();

    const columns: GridColDef[] = [
        {
            field: "status",
            headerName: "Status",
            width: 140,
            editable: false,
            cellClassName: (params: GridCellParams<string>) => {
                if (params.value == null) {
                    return '';
                }

                return clsx('super-app', {
                    deployed: params.value == "Deployed",
                    notdeployed: params.value == "Not Deployed",
                    unhealthy: params.value == "Unhealthy",
                });
            },
        },
        {
            field: "analyzerName",
            headerName: "Analyzer Name",
            width: 300,
            editable: false,
        },
        {
            field: "date",
            headerName: "Date",
            width: 250,
            editable: false,
        },
    ];

    const rows = [
        { id: 1, status: "Deployed", analyzerName: "Suspicious svchost", date: "07/17/22" },
        { id: 2, status: "Not Deployed", analyzerName: "Weird Stuff", date: "08/13/22" },
        { id: 3, status: "Unhealthy", analyzerName: "Evil Files", date: "08/17/22" },
    ];

    return (
        <div className={classes.generatorsListTable}>
            <Box
                sx={{
                    height: 300,
                    width: '100%',
                    '& .super-app-theme--cell': {
                        backgroundColor: 'rgba(224, 183, 60, 0.55)',
                        color: '#212936',
                        fontWeight: '600',
                    },
                    '& .super-app.deployed': {
                        backgroundColor: 'rgba(157, 255, 118, 0.49)',
                        color: '#212936',
                        fontWeight: '600',
                    },
                    '& .super-app.notdeployed': {
                        backgroundColor: '#CCCCCC',
                        color: '#212936',
                        fontWeight: '600',
                    },
                    '& .super-app.unhealthy': {
                        backgroundColor: '#d47483',
                        color: '#1a3e72',
                        fontWeight: '600',
                    },
                }}
            >
            <DataGrid
                sx={{
                    bgcolor: "#212936",
                    color: "#FFF",
                    boxShadow: 1,
                    border: 0,
                    borderRadius: 2,
                    p: 2,
                    minWidth: 800,
                    "& 	.MuiDataGrid-columnHeader": {
                        color: "#8997B1",
                    },
                    "& .MuiDataGrid-columnSeparator": {
                        visibility: "hidden",
                    },
                    "& .MuiDataGrid-sortIcon": {
                        color: "#8997B1",
                    },
                    "& .MuiCheckbox-root": {
                        color: "#8997B1",
                    },
                    "& .MuiIconButton-root": {
                        color: "#8997B1",
                    },
                    "& .MuiTablePagination-displayedRows": {
                        color: "#8997B1",
                    },
                    "& .MuiTablePagination-actions": {
                        color: "#8997B1",
                    },
                }}
                rows={rows}
                columns={columns}
                pageSize={5}
                rowsPerPageOptions={[5]}
                checkboxSelection
                disableSelectionOnClick
            />
            </Box>
        </div>
    );
};

const NewAnalyzers = () => {
    const classes = useStyles();

    return (
        <Box className={classes.root} sx={{ display: "flex" }}>
            <NavigationDrawer />

            {/*<div className={classes.metricsAndUploadContainer}>*/}

            {/*    <UploadForm></UploadForm>*/}
            {/*</div>*/}

            <div>
                <UploadPluginModal />
            </div>
            <div>
                <AnalyzersListTable></AnalyzersListTable>
            </div>
        </Box>
    );
};

export default NewAnalyzers;
