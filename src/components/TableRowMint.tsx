import { useState } from "react";
import { Rating, Table, Tooltip } from "flowbite-react";
import { BsClipboard2, BsClipboard2CheckFill, BsTrash } from "react-icons/bs";
import { Nip87MintInfo } from "@/types";
import { useNdk } from "@/hooks/useNdk";
import { useDispatch, useSelector } from "react-redux";
import { RootState } from "@/redux/store";
import { deleteMintInfo } from "@/redux/slices/nip87Slice";
import ReviewMintButton from "./ReviewMintButton";
import { copyToClipboard, shortenString } from "@/utils";

const TableRowMint = ({ mint }: { mint: Nip87MintInfo }) => {
  const [copied, setCopied] = useState(false);
  const [show, setShow] = useState(false);

  const user = useSelector((state: RootState) => state.user);

  const { ndk, attemptDeleteEvent } = useNdk();

  const dispatch = useDispatch();

  const handleDelete = async () => {
    attemptDeleteEvent(mint.rawEvent);

    const mintInfoId = `${mint.mintUrl}${mint.appPubkey}`;
    dispatch(deleteMintInfo(mintInfoId));
  };

  const handleModalClose = () => {
    setShow(false);
  };

  const handleCopy = () => {
    copyToClipboard(mint.mintUrl);
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
  };

  return (
    <>
      <Table.Row className="dark:bg-gray-800">
        {/* Mint name */}
        <Table.Cell>{mint.mintName}</Table.Cell>

        {/* Average Rating */}
        <Table.Cell>
          {mint.totalRatings ? (
            <Rating>
              <Rating.Star />
              &nbsp;{mint.totalRatings / mint.numRecsWithRatings || "N/A"}
              &nbsp;&middot;&nbsp;
              {mint.numRecsWithRatings} review{mint.numRecsWithRatings > 1 ? "s" : ""}
            </Rating>
          ) : (
            "N/A"
          )}
        </Table.Cell>

        {/*  Mint Url */}
        <Table.Cell className="hover:cursor-pointer" onClick={handleCopy}>
          <div className="flex">
            {shortenString(mint.mintUrl)}
            {copied ? (
              <BsClipboard2CheckFill className="ml-1 mt-1" size={15} />
            ) : (
              <BsClipboard2 className="ml-1 mt-1" size={15} />
            )}
          </div>
        </Table.Cell>
        <Table.Cell>{mint.supportedNuts || "N/A"}</Table.Cell>
        <Table.Cell>
          {user.pubkey === mint.appPubkey ? (
            <Tooltip content="Attempt to delete">
              <BsTrash
                onClick={handleDelete}
                className="h-6 w-6 text-red-600 hover:cursor-pointer"
              />
            </Tooltip>
          ) : (
            <ReviewMintButton mint={mint} text="Add Review" />
          )}
        </Table.Cell>
      </Table.Row>
    </>
  );
};

export default TableRowMint;
