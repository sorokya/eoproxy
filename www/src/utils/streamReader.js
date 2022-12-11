import { decodeNumber } from "./decodeNumber";

export class StreamReader {
  constructor (data) {
    this.data = data;
    this.position = 2;
  }

  eof() {
    return this.position >= this.data.length;
  }

  getByte() {
    let byte = this.data[this.position];
    this.position += 1;
    return byte;
  }

  getChar() {
    const char = this.data[this.position];
    this.position += 1;
    return decodeNumber([char]);
  }

  getShort() {
    const short = this.data.slice(this.position, this.position + 2);
    this.position += 2;
    return decodeNumber(short);
  }

  getThree() {
    const three = this.data.slice(this.position, this.position + 3);
    this.position += 3;
    return decodeNumber(three);
  }

  getInt() {
    const int = this.data.slice(this.position, this.position + 4);
    this.position += 4;
    return decodeNumber(int);
  }

  getFixedString(length) {
    const encoder = new TextDecoder();
    const string = encoder.decode(new Uint8Array(this.data.slice(this.position, this.position + length)));
    this.position += length;
    return string;
  }

  getBreakString() {
    const breakIndex = this.data.indexOf(255, this.position);
    const string = this.getFixedString(breakIndex - this.position);
    this.position += 1;
    return string;
  }

  getPrefixString() {
    const length = this.getChar();
    return this.getFixedString(length);
  }

  getEndString() {
    return this.getFixedString(this.data.length - this.position);
  }

  peekByte() {
    return this.data[this.position];
  }
}