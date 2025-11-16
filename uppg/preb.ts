async function* generator() {
  let n = 0
  while (n < 18) {
    n++
    yield n
  }
}
const item = generator()
for (let i = 0; i < 20; i++) {
  const this_iter = item.next()
  if (this_iter.done) {
    console.log("done!")
    break
  }
  console.log(this_iter.value)
}
