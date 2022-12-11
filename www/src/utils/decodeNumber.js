const MAX1 = 253;
const MAX2 = 64009;
const MAX3 = 16194277;

export function decodeNumber(bytes) {
  const data = [254, 254, 254, 254];
  for(let i = 0; i < 4; ++i) {
    if (bytes.length > i && bytes[i] !== 0) {
      data[i] = bytes[i];
    }

    if (data[i] === 254) {
      data[i] = 1;
    }

    data[i] -= 1;
  }

  return data[3] * MAX3 + data[2] * MAX2 + data[1] * MAX1 + data[0];
}