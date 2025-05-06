import { createEffect, onMount, ParentProps } from "solid-js";

type ModalProps = ParentProps & {
  open: boolean;
  onClose: () => void;
};

export default function Modal(props: ModalProps) {
  let dialogRef: HTMLDialogElement | null = null;

  onMount(() => {
    dialogRef?.addEventListener("close", () => {
      props.onClose();
    });
  });

  createEffect(() => {
    if (props.open) {
      if (!dialogRef?.open) {
        dialogRef?.showModal();
      }
    } else {
      if (dialogRef?.open) {
        dialogRef?.close();
      }
    }
  });

  return (
    <dialog
      class="m-auto"
      ref={(el) => (dialogRef = el)}
      // onClose={props.onClose}
    >
      {props.children}
    </dialog>
  );
}
