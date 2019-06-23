// Stylesheets

console.log('Loaded index.js');

const engagement_edge = "https://dzuo4ozud3.execute-api.us-east-1.amazonaws.com/prod/";

const getLenses = async () => {
    const res = await fetch(`${engagement_edge}getLenses`, {
        method: 'post',
        body: JSON.stringify({
            'prefix': '',
        })
    });

    return await res.json();
};

const nodeToTable = (lens) => {

    let header = '<thead class="thead"><tr>';
    let output = '<tbody><tr>';
    header += `<th scope="col">lens</th>`;
    header += `<th scope="col">score</th>`;
    header += `<th scope="col">link</th>`;

    output += `<td>${lens.lens}</td>>`;
    output += `<td>${lens.score}</td>>`;
    // output += `<td><a href="${engagement_edge}lens.html?lens=${lens.lens}">link</td></a>>`;
    output += `<td><a href="lens.html?lens=${lens.lens}">link</td></a>>`;


    return `${header}</tr></thead>` + `${output}</tr><tbody>`;
};


document.addEventListener('DOMContentLoaded', async (event) => {
    console.log('DOMContentLoaded');

    const lenses = (await getLenses()).lenses;
    console.log(lenses.lenses);

    const s = nodeToTable(lenses[0]);

    const lenseTable = document.getElementById('LenseTable');

    lenseTable.innerHTML = `<table>${s}</table>`;


});