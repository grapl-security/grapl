import React, { useEffect } from "react";

export function useEffectUponMount(fn: () => any) {
    // a React Hooks equivalent to componentDidMount.
    // Runs an effect once and only once.
    // https://medium.com/@felippenardi/how-to-do-componentdidmount-with-react-hooks-553ba39d1571
    useEffect(fn, []);
}
