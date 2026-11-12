// Corner Shop cart. State lives in memory; this is a demo target for Reenact.
const PRODUCTS = [
  { id: 'beans', name: 'Coffee beans (250 g)', price: 12 },
  { id: 'mug', name: 'Enamel mug', price: 8 },
  { id: 'filters', name: 'Paper filters (100)', price: 5 },
];

const state = { lines: [], discount: 0 };

const $ = (id) => document.getElementById(id);
const money = (n) => (n < 0 ? '-$' : '$') + Math.abs(n).toFixed(2);

function addToCart(id) {
  const line = state.lines.find((l) => l.id === id);
  if (line) line.qty += 1;
  else state.lines.push({ id, qty: 1 });
  render();
}

function changeQty(id, delta) {
  const line = state.lines.find((l) => l.id === id);
  if (!line) return;
  line.qty += delta;
  render();
}

function removeLine(index) {
  state.lines.splice(index, 1);
  render();
}

function subtotal() {
  return state.lines.reduce((sum, l) => sum + PRODUCTS.find((p) => p.id === l.id).price * l.qty, 0);
}

function applyCoupon(code) {
  if (code.trim().toUpperCase() !== 'SAVE10') {
    state.discount = 0;
    $('coupon-msg').textContent = 'That code is not valid.';
  } else {
    const items = state.lines.reduce((n, l) => n + l.qty, 0);
    state.discount = items > 1 ? -10 : 10;
    $('coupon-msg').textContent = 'Coupon SAVE10 applied.';
  }
  render();
}

function render() {
  $('products').innerHTML = PRODUCTS.map(
    (p) => `<li><span>${p.name} <small>${money(p.price)}</small></span><button data-add="${p.id}">Add ${p.name.split(' ')[0].toLowerCase()}</button></li>`
  ).join('');
  const lines = [...state.lines].sort((a, b) => a.id.localeCompare(b.id));
  $('empty').hidden = state.lines.length > 0;
  $('cart').innerHTML = lines
    .map((l, i) => {
      const p = PRODUCTS.find((x) => x.id === l.id);
      return `<li><span>${p.name}</span><span class="qty"><button data-dec="${l.id}" aria-label="Decrease ${p.name}">-</button><output>${l.qty}</output><button data-inc="${l.id}" aria-label="Increase ${p.name}">+</button></span><button data-remove="${i}">Remove</button></li>`;
    })
    .join('');
  const sub = subtotal();
  $('subtotal').textContent = money(sub);
  $('discount').textContent = money(-state.discount);
  $('total').textContent = money(sub - state.discount);
}

document.addEventListener('click', (e) => {
  const t = e.target.closest('button');
  if (!t) return;
  if (t.dataset.add) addToCart(t.dataset.add);
  if (t.dataset.inc) changeQty(t.dataset.inc, 1);
  if (t.dataset.dec) changeQty(t.dataset.dec, -1);
  if (t.dataset.remove !== undefined) removeLine(Number(t.dataset.remove));
});

$('coupon-form').addEventListener('submit', (e) => {
  e.preventDefault();
  applyCoupon($('coupon').value);
});

render();
