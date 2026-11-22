/*! Original Thinkwell experiment fixture; MIT OR Apache-2.0. */
function a(xs) { return xs.reduce((total, x) => total + /* add value */ x, 0); }
export function average(xs) { return xs.length ? a(xs) / xs.length : 0; }
