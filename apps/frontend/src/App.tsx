import { createSignal, type JSX } from "solid-js";

export default function App(): JSX.Element {
  const [number, setNumber] = createSignal(0);

  return (
    <>
      <button
        id="increment"
        name="increment"
        onClick={() => setNumber(number() + 1)}
      >
        Increment: {number()}
      </button>
      <button
        id="decrement"
        name="decrement"
        onClick={() => setNumber(number() - 1)}
      >
        Decrement: {number()}
      </button>
    </>
  );
}
