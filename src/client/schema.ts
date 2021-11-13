import {Schema, serialize, deserializeUnchecked} from 'borsh';
import BN from 'bn.js';
import {Struct, Enum, PublicKey, SOLANA_SCHEMA} from '@solana/web3.js';

import * as borsh from 'borsh';

/**
 * The state of a greeting account managed by the hello world program
 */
export class ChannelAccount {
    name:string = "New channel";
    tail_message:PublicKey|undefined = undefined
    constructor(fields: {name: string, tail_message?: PublicKey} | undefined = undefined) {
        if (fields) {
        this.name = fields.name;
        this.tail_message = fields.tail_message
        }
    }
}


export const addSolchatSchema = (schema: Schema) =>  {
    /**
     * Borsh requires something called a Schema,
     * which is a Map (key-value pairs) that tell borsh how to deserialise the raw data
     * This function adds a new schema to an existing schema object.
     */
    schema.set(PublicKey, {
        kind: 'struct',
        fields: [['_bn', 'u256']],
    });
    
    schema.set(ChannelAccount, {
        kind: 'struct',
        fields:  [['name', 'string'],['tail_message', { kind: 'option', type: PublicKey }]],
    });

    return schema
    
}
export const SOLCHAT_SCHEMA = addSolchatSchema(SOLANA_SCHEMA) 



/**
 * The expected size of each greeting account.
 */
export const CHANNEL_ACCOUNT_SIZE = borsh.serialize(
    SOLCHAT_SCHEMA,
    new ChannelAccount(),
).length;


