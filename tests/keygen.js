/**
 * @type {string}
 */
const keygenKey = __TEMPLATE_key__

/**
 * @type {number}
 */
const keygenLength = __TEMPLATE_length__

__RAW_optional_script__

// app logic, we want to make sure the key value's length equals the `Keygen` template's expected length
if (keygenKey.length === keygenLength) {
    console.log("okay!")
} else {
    console.error("oh no!")
}