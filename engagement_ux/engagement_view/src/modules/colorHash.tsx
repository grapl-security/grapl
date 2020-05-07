const BKDRHash = (str: any) => {
    const seed = 131;
    const seed2 = 137;
    let hash = 0 as any;
    // make hash more sensitive for short string like 'a', 'b', 'c'
    str += 'x';
    // Note: Number.MAX_SAFE_INTEGER equals 9007199254740991
    const MAX_SAFE_INTEGER = parseInt(9007199254740991 / seed2 as any) as any;
    for (let i = 0; i < str.length; i++) {
        if (hash > MAX_SAFE_INTEGER) {
            hash = parseInt(hash / seed2 as any);
        }
        hash = hash * seed + str.charCodeAt(i);
    }
    return hash;
};


export { BKDRHash } 