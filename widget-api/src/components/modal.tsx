import { createEffect, ParentProps } from "solid-js";

type ModalProps = ParentProps & {
  open: boolean;
};

export default function Modal(props: ModalProps) {
  let dialogRef: HTMLDialogElement | null = null;

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

  return <dialog ref={(el) => (dialogRef = el)}>{props.children}</dialog>;
}
