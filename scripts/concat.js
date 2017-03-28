function concat(str1, str2) {
  return str1 + str2;
}

function concatPromise(str1, str2) {
  return Promise.resolve(concat(str1, str2));
}
