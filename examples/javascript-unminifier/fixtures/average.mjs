/*! Original Thinkwell example fixture; MIT OR Apache-2.0. */
function a(xs) { return xs.reduce((total, x) => total + /* add value */ x, 0); }
function b(xs) { return xs.length; }
export function average(xs) { return b(xs) ? a(xs) / b(xs) : 0; }
