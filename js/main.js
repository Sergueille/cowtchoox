
// This file will create nice page elements, and fil them with the content of the document
// This allows to place headers and to have specific rule for page layout
// It's currently a total mess...


console.log("Hello world from JS!")
main();


// FIXME: this will create an infinite loop if one element is too big to fit one one page
//        please fix this
async function main() {
    // Gather all document elements
    let children = Array.from(document.body.children);
    children.reverse(); // Reverse it to pop th elements one by one later

    // Removes them all
    document.body.innerText = "";

    let pageNumber = 1;
    while (true) {
        let pageElement = getPage(pageNumber);
        document.body.appendChild(pageElement);

        // Add elements until the page overflows
        while (children.length > 0) {
            let top = children.pop();

            pageElement.appendChild(top);
            
            // Wait to make sure the browser have updated the layout
            await new Promise(resolve => setTimeout(resolve, 0));

            if (isOverflowing(pageElement)) { // The page is full // TODO: cut the element
                top.remove(top);
                children.push(top)
                break;
            }
        }

        if (children.length == 0) break; // No more elements
        pageNumber++;
    }
}


/**
 * Creates a new page HTML element
 * @param {number} pageNumber The id of the page (starting from 1)
 * @returns The brand new page
 */
function getPage(pageNumber) {
    let [pageWidth, pageHeight] = getPageSize();

    let res = document.createElement("page");
    res.setAttribute("id", "page-" + pageNumber.toString());
    res.style.width = pageWidth + "mm";
    res.style.height = pageHeight + "mm";

    return res;
}


/**
 * Fetches the page size from the document's head
 * @returns (width, height), in mm
 */
function getPageSize() {
    return [
        +document.head.querySelector('meta[name="pagewidth"]').content,
        +document.head.querySelector('meta[name="pageheight"]').content
    ];
}


/**
 * Check if the content of an element content is overflowing (https://stackoverflow.com/questions/143815/determine-if-an-html-elements-content-overflows)
 * @param {HTMLElement} el The element
 */
function isOverflowing(el) {
    return el.clientHeight < el.scrollHeight;
}

