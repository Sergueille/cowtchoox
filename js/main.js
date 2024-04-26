// This file will create nice page elements, and fil them with the content of the document
// This allows to place headers and to have specific rules for page layout

// Tags that have default properties by default by default
const defaultNonbreaking = [
    "H1", "H2", "H3", "H4", "H5", "H6", "SVG", "PRE", "AMP-SPLIT", "DOUBLE-AMP-SPLIT"
];
const defaultStickafter = [
    "H1", "H2", "H3", "H4", "H5", "H6",
];


/** Errors will be stored here until cowtchoox evaluate this to show them in the terminal
 * @type{Array<String>}
 */
let errors = [];


main().then(() => {}).catch(err => {
    errors.push(err.message);
    console.error(err);
    createErrorElement();
});


async function main() {
    let footer = findFooter();
    let header = findHeader();

    replaceEvaluate(document);
    replaceLastValues(document);

    // Gather all document elements
    let children = Array.from(document.body.children);

    // Removes them all
    document.body.innerText = "";

    let pageNumber = 1;
    while (true) {
        let pageElement = getPage(pageNumber);
        document.body.appendChild(pageElement);

        if (header) {
            let instance = header.cloneNode(true);
            replacePageNumbers(instance, pageNumber);
            replaceEvaluate(instance);
            replaceLastValues(instance);
            pageElement.appendChild(instance);
        }

        let insidePage = document.createElement("page-inside");
        pageElement.appendChild(insidePage);

        if (footer) {
            let instance = footer.cloneNode(true);
            replacePageNumbers(instance, pageNumber);
            replaceEvaluate(instance);
            replaceLastValues(instance);
            pageElement.appendChild(instance);
        }

        for (let child of children) {
            insidePage.appendChild(child);
        }

        replacePageNumbers(insidePage, pageNumber);

        let [remaining, addedSomething] = await fillUntilOverflow(pageElement, insidePage);

        // Remove scrollbars that may have appeared and prevent bugs related to the page becoming slightly less wide
        pageElement.style.setProperty("overflow", "hidden");

        if (remaining == null) break; // No more elements

        // Nothing was added, because the next nonbreaking element is too large...
        // To prevent endless loops, force-add the next element and report a warning 
        if (!addedSomething) {
            let firstChild = remaining.children[0];
            remaining.removeChild(firstChild);
            pageElement.appendChild(firstChild);

            logError(`A nonbreaking element (${firstChild.tagName}) is too large to fit in the page.`);
        }

        children = Array.from(remaining.children);

        pageNumber++;
    }

    createErrorElement();
}


/**
 * Will replace all values of evaluate tags
 @param {HTMLElement} parent
 */
function replaceEvaluate(parent) {
    // Search for get tags
    for (let tag of parent.querySelectorAll("evaluate > inner")) {
        let expression = tag.textContent;

        try {
            let result = eval(expression);
            tag.innerHTML = result;
        } catch (err) {
            logError(`Failed to parse the evaluate tag that contains "${tag.textContent}". The error is: "${err.message}"`)
            tag.innerHTML = "Evaluation failed.";
        }
    }
}


/**
 * Will replace all values of last-tag-value tags
 @param {HTMLElement} parent
 */
function replaceLastValues(parent) {
    // Search for get tags
    for (let tag of parent.querySelectorAll("last-tag-value > span > text")) {
        let tagName = tag.innerHTML;

        let lastId = tag.id;
        tag.id = "replace-temp-id" // Creates a temporary id to match this tag later

        let finalContent = "";

        // Query all tag with this name, including this tag
        let nodeList = document.querySelectorAll(tagName, "#replace-temp-id");
        for (let node of nodeList) {
            if (node == tag) {
                break;
            } else {
                finalContent = node.innerHTML;
            }
        }

        tag.innerHTML = finalContent;
        tag.id = lastId;
    }
}


/**
 * Will replace all values of page number tags in parent with provided value
 * @param {HTMLElement} parent
 * @param {number} page
 */
function replacePageNumbers(parent, page) {
    // Search for get tags
    for (let tag of parent.querySelectorAll("page-number")) {
        tag.innerHTML = page.toString();
    }
}

/**
 * Inserts an element into parentElement, inside pageElement, and cuts it if necessary
 * @param {HTMLElement} pageElement 
 * @param {HTMLElement} parentElement 
 * @returns {Promise<[HTMLElement, bool]>} The rest of the element that couldn't be inserted in the page. Returns null if everything were inserted. 
 *                                         The boolean is true if at least a part of an element was inserted
 */
async function fillUntilOverflow(pageElement, parentElement) {
    let children = Array.from(parentElement.children);
    children.reverse();

    let addedPartOfElement = false;
    let removeStickbefore = false;
    let addedElements = [];

    parentElement.innerText = ""; // Remove children

    // Add elements until the page overflows
    while (children.length > 0) {
        let top = children.pop();

        // Handle page break
        if (top.tagName == "PAGEBREAK") {
            addedPartOfElement = true;
            break;
        } else if (top.querySelector("pagebreak") != null) {
            if (isNonbreaking(top)) {
                logError(`There is a pagebreak inside a nonbreaking element (${top.tagName})`);
            }

            parentElement.appendChild(top);
            let [remaining, addedPartOfTop] = await fillUntilOverflow(pageElement, top);
            addedPartOfElement = addedPartOfElement || addedPartOfTop;

            if (remaining != null) {
                top.classList.add("first-half");
                remaining.classList.add("second-half");
                children.push(remaining);
            }

            break;
        }

        parentElement.appendChild(top);

        // Wait to make sure the browser have updated the layout
        await new Promise(resolve => setTimeout(resolve, 0));

        if (isOverflowing(pageElement)) { // The page is full
            if (isStickbefore(top)) {
                removeStickbefore = true;
                parentElement.removeChild(top);
                children.push(top);
            } else if (isNonbreaking(top)) { // Finished!
                parentElement.removeChild(top);
                children.push(top);
            } else if (top.tagName == "TEXT") { // Split text
                let text = top.textContent;
                top.textContent = "";

                // Overflowing even if empty
                if (isOverflowing(pageElement)) {
                    parentElement.removeChild(top);
                    top.textContent = text;
                    children.push(top);
                } else {
                    let word = "";
                    let wordStartId = 0;
                    for (let i = 0; i < text.length; i++) {
                        let ch = text[i];
                        if (/\s/.test(ch)) { // whitespace: try to cut!
                            top.textContent += word;

                            if (isOverflowing(pageElement)) {
                                top.textContent = top.textContent.slice(0, top.textContent.length - word.length);
                                break;
                            }

                            addedPartOfElement = true;
                            word = "";
                            wordStartId = i;
                        }

                        word += ch;
                    }

                    let secondHalf = top.cloneNode(false);
                    secondHalf.textContent = text.slice(wordStartId, text.length);
                    children.push(secondHalf);

                    top.classList.add("first-half");
                    secondHalf.classList.add("second-half");
                }
            } else { // Split element
                parentElement.removeChild(top);

                let cloned = top.cloneNode(false);
                cloned.innerHTML = "";
                parentElement.appendChild(cloned);

                // Overflows even if empty: put all on next page
                if (isOverflowing(pageElement)) {
                    parentElement.removeChild(cloned);
                    children.push(top);
                } else {
                    parentElement.removeChild(cloned);
                    parentElement.appendChild(top);

                    let [remaining, addedPartOfTop] = await fillUntilOverflow(pageElement, top);
                    addedPartOfElement = addedPartOfElement || addedPartOfTop;

                    if (remaining != null) {
                        if (addedPartOfTop) {
                            top.classList.add("first-half");
                            remaining.classList.add("second-half");
                            children.push(remaining);
                        } else { // Nothing was added inside, revert the operation 
                            parentElement.removeChild(top);

                            // Put the children back
                            while (remaining.children.length > 0) {
                                top.appendChild(remaining.children[0]);
                            }

                            children.push(top);
                        }
                    }
                }
            }

            break;
        } else {
            addedElements.push(top);
        }
    }

    if (!addedPartOfElement && children.length > 0) {
        // Also remove stickafter elements
        while (addedElements.length > 0 && isStickafter(addedElements[addedElements.length - 1])) {
            let last = addedElements.pop();
            parentElement.removeChild(last);
            children.push(last);
        }
    }

    if (removeStickbefore) {
        // Remove stickbefore elements, and the next element
        while (addedElements.length > 0 && isStickbefore(addedElements[addedElements.length - 1])) {
            let last = addedElements.pop();
            parentElement.removeChild(last);
            children.push(last);
        }

        if (addedElements.length > 0) {
            let last = addedElements.pop();
            parentElement.removeChild(last);
            children.push(last);
        }
    }

    if (children.length == 0) {
        return [null, addedElements.length > 0 || addedPartOfElement];
    } else {
        let copy = parentElement.cloneNode(false);
        copy.innerText = "";

        children.reverse();
        for (let child of children) {
            copy.appendChild(child);
        }

        return [copy, addedElements.length > 0 || addedPartOfElement];
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


/**
 * Determines if the element is nonbreaking
 * @param {HTMLElement} tag
 * @returns {Boolean}
 */
function isNonbreaking(tag) {
    return tag.getAttribute("nonbreaking") != null || defaultNonbreaking.includes(tag.tagName) || isStickafter(tag) || isStickbefore(tag);
}


/**
 * Determines if the element is stickafter
 * @param {HTMLElement} tag
 * @returns {Boolean}
 */
function isStickafter(tag) {
    return tag.getAttribute("stickafter") != null || defaultStickafter.includes(tag.tagName);
}


/**
 * Determines if the element is stickbefore
 * @param {HTMLElement} tag
 * @returns {Boolean}
 */
function isStickbefore(tag) {
    return tag.getAttribute("stickbefore") != null;
}


/**
 * Crates an element 
 * @param {String} message
 */
function logError(message) {
    errors.push(message);
}


/**
 * Creates an element on the page so that cowtchoox can show the errors in the log
 */
function createErrorElement() {
    let el = document.createElement("div");
    el.id = "cowtchoox-error-reporter"
    el.style.display = "none";
    el.textContent = errors.join("\0");

    document.body.appendChild(el);
}


/**
 * Find the footer element in the document and removes it
 * @returns {HTMLelement} The footer
 */
function findFooter() {
    let res = document.querySelector("doc-footer");

    if (res) {
        res.parentElement.removeChild(res);
    }

    return res;
}


/**
 * Find the header element in the document and removes it
 * @returns {HTMLelement} The footer
 */
function findHeader() {
    let res = document.querySelector("doc-header");

    if (res) {
        res.parentElement.removeChild(res);
    }

    return res;
}