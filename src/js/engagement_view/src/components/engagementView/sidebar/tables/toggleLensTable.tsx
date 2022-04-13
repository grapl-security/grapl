import React, { useEffect, useState } from "react";
import { useEffectUponMount } from "lib/custom_react_hooks";

import Button from "@material-ui/core/Button";
import KeyboardArrowDownOutlinedIcon from "@material-ui/icons/KeyboardArrowDownOutlined";
import KeyboardArrowUpOutlinedIcon from "@material-ui/icons/KeyboardArrowUpOutlined";

import Divider from "@material-ui/core/Divider";
import Backdrop from "@material-ui/core/Backdrop";
import CircularProgress from "@material-ui/core/CircularProgress";

import { lensTable } from "./lensTable/lensTable";
import { getLenses } from "services/graphQLRequests/getLenses";

import { ToggleLensTableProps, ToggleLensTableState } from "types/CustomTypes";

import { useStyles } from "./lensTable/lensTableStyles";

const defaultToggleLensTableState = (): ToggleLensTableState => {
    return {
        toggled: true,
        lenses: [],
        first: 100, // first is the page size
        offset: 0, // by default, start from page 0
    };
};

export function ToggleLensTable({ setLens }: ToggleLensTableProps) {
    const classes = useStyles();

    const [lensRetrievedState, setLensRetrievedState] = useState(null);
    const [toggleTableState, setToggleTableState] = useState(
        defaultToggleLensTableState()
    );
    const [pageState, setPageState] = useState(0);
    const [rowsPerPageState, setRowsPerPageState] = useState(10);

    const handleChangePage = (
        event: React.MouseEvent<HTMLButtonElement, MouseEvent> | null,
        page: number
    ) => {
        setPageState(page);
    };

    const handleChangeRowsPerPage = (
        event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
    ) => {
        setRowsPerPageState(parseInt(event.target.value, 10));
        setPageState(0);
    };

    async function updateTableStateWithLenses() {
        const response = await getLenses(
            toggleTableState.first,
            toggleTableState.offset
        );
        if (response.lenses && response.lenses !== toggleTableState.lenses) {
            const lenses = toggleTableState.lenses.concat(response.lenses);
            setLensRetrievedState(lenses as any);
            setToggleTableState({
                ...toggleTableState,
                offset: toggleTableState.offset + response.lenses.length || 0,
                lenses,
            });
        }
    }
    // Do an initial updateTableStateWithLenses just once.
    useEffectUponMount(updateTableStateWithLenses);

    // Schedule an updateTableStateWithLenses every N seconds
    useEffect(() => {
        const interval = setInterval(updateTableStateWithLenses, 5000);
        return () => clearInterval(interval);
    });

    return (
        <>
            <div className={classes.header}>
                <b className={classes.title}> Lenses </b>
                <Button
                    className={classes.lensToggleBtn}
                    onClick={() => {
                        setToggleTableState({
                            ...toggleTableState,
                            toggled: !toggleTableState.toggled,
                        });
                    }}
                >
                    {toggleTableState.toggled ? (
                        <KeyboardArrowUpOutlinedIcon
                            className={classes.expand}
                        />
                    ) : (
                        <KeyboardArrowDownOutlinedIcon
                            className={classes.expand}
                        />
                    )}
                </Button>
            </div>

            <div className="lensToggle">
                {lensRetrievedState ? (
                    toggleTableState.toggled &&
                    lensTable(
                        toggleTableState,
                        pageState,
                        rowsPerPageState,
                        handleChangePage,
                        handleChangeRowsPerPage,
                        setLens,
                        classes
                    )
                ) : (
                    <Backdrop className={classes.backdrop} open>
                        <CircularProgress color="inherit" />
                    </Backdrop>
                )}
            </div>

            <Divider />
        </>
    );
}
